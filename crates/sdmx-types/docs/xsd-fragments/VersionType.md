<details>
<summary>XSD contract: <code>VersionType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="VersionType">
		<xs:annotation>
			<xs:documentation>VersionType is used to communicate version information. Semantic versioning, based on 3 or 4 version parts (major.minor.patch[-extension]) is supported. The legacy SDMX version format is also supported.</xs:documentation>
		</xs:annotation>
		<xs:union memberTypes="LegacyVersionNumberType SemanticVersionNumberType"/>
	</xs:simpleType>
```

</details>
