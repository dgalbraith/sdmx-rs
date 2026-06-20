<details>
<summary>XSD contract: <code>ProvisionAgreementReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="ProvisionAgreementReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a provision agreement.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="MaintainableUrnReferenceType">
			<xs:pattern value=".+\.registry\.ProvisionAgreement=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
