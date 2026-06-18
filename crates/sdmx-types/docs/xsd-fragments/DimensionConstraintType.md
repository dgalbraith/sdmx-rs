<details>
<summary>XSD contract: <code>DimensionConstraintType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="DimensionConstraintType">
		<xs:annotation>
			<xs:documentation>Specifies the fixed list of Dimensions (by ID) to which a Dataflow may be constrained. This is a required property if the DataStructure defines itself as an evolving structure, indicating that it can change dimensionality under a minor version change, and if the Dataflow references that DataStructure using a wildcarded minor version number. New minor DSD version can so still be used by this Dataflow even if that DSD version defines new additional dimensions. Dimensions not listed should not be used in Datasets for this Dataflow. The Time Dimension is not to be listed, as it is always to be used when defined in the DSD, and it cannot be added to a DSD without increasing its major version.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="Dimension" type="common:IDType" minOccurs="1" maxOccurs="unbounded" />
		</xs:sequence>
	</xs:complexType>
```

</details>
